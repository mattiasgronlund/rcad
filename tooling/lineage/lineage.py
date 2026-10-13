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

No row filed before Q1703 merged could say `from/fuzz:` or `from/step:`, so
a fault a night or a corpus file showed hangs there on whatever row was
running, or on none. Where such a row's own text names the input that shows
its fault -- a crash file, a seed by its hash, or one corpus file -- that
fuzz target or corpus file is its parent instead (§433.6, §433.2), unless the
row it already hangs on is in that input's tree. Every table that reads a
parent is printed twice, without those sources and with them, and the
sources read from text are listed on their own.

Deliberately not `import queue`: `tooling/queue/queue.py` as a module named
`queue` would shadow the standard library's own `queue`, which subprocess and
threading can depend on. Only `Row` and `Queue` come from `rows.py`, which
collides with nothing. `CAME_FROM` and `CAME_FROM_AN_INPUT` are duplicated
from `queue.py` rather than imported for the same reason.

Parsing the file at each commit is what costs a walk its minutes, so each
commit is read once, into a summary of the rows it added and the rows it
closed (`summarize`), and the rules above read only those. The queue page
keeps the summaries between renders and parses only the commits that are new
since (§444).
"""

import bisect
import functools
import os
import re
import subprocess
import sys
from collections import defaultdict
from datetime import datetime, timedelta, timezone

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "queue"))
from rows import CLASS, PIN, SMALL, Queue  # noqa: E402

QUEUE_PATH = "docs/queue.md"
MANIFEST_PATH = "corpus/manifest.sha256"
TARGETS_DIR = "fuzz/fuzz_targets/"
SEEDS_DIR = "fuzz/corpus/"
FIELD_SEP = "\x1f"
RECORD_SEP = "\x1e"

CAME_FROM = re.compile(r"^(from|was)/([TDQ][0-9]+|-)$")
CAME_FROM_AN_INPUT = re.compile(r"^from/(fuzz|step):(\S+)$")
LANE_MERGE_SUBJECT = re.compile(r"^Merge lane-q([0-9]+)-")
ROW_ID = re.compile(r"^Q[0-9]+$")

# The row whose merge let a row's word name a fuzz target or a STEP file (§433.7).
ROW_THAT_LET_A_WORD_NAME_AN_INPUT = "Q1703"
CRASH_FILE_UNDER_ITS_TARGET = re.compile(r"artifacts/([A-Za-z0-9_]+)/crash-[0-9a-f]+")
CRASH_FILE_ALONE = re.compile(r"(?<![A-Za-z0-9_/])crash-[0-9a-f]{6,}")
HASH = re.compile(r"(?<![A-Za-z0-9])[0-9a-f]{8,40}(?![A-Za-z0-9])")
STEP_FILE_NAME = re.compile(r"(?<![A-Za-z0-9_.\-])[A-Za-z0-9_.\-]+\.(?:stp|step)(?![A-Za-z0-9_\-])")
MANIFEST_LINE = re.compile(r"^[0-9a-f]{64} [ *](.+)$")

# The shape of what `summarize` returns, so a summary kept under another shape
# is never read as this one. Bump it whenever `summarize`, or `rows_at` that it
# reads through, changes what a commit's summary holds.
SUMMARY_SHAPE = 1


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


def load_blobs(top, shas, path):
    """`path`'s bytes at every sha in `shas`, `None` where it does not exist there.

    One `git cat-file --batch` rather than one `git show` per sha, for the
    corpus manifest at every commit that touched it. Output is read by the
    byte count each header gives, not split on newlines, since the general
    shape of a blob is arbitrary.
    """
    ids = sorted(set(shas))
    if not ids:
        return {}
    stdin = "".join(f"{sha}:{path}\n" for sha in ids).encode()
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


def summarize_missing(top, commits, summaries):
    """Add to `summaries` the summary of every commit in `commits` it lacks.

    Each copy of `docs/queue.md` is fetched by its blob id and parsed once
    however many commits hold it: on `main` at 3ca67d02, 3472 commits and
    parents hold 2296 distinct copies, and parsing one takes about 0.09 s.
    A few are kept parsed, since a commit's parents are mostly the commits just
    before it in this walk, and only a few, since one copy parsed holds
    every row of the file.
    """
    missing = [commit for commit in commits if commit.sha not in summaries]
    if not missing:
        return
    blob_of = blob_ids(top, {sha for commit in missing for sha in (commit.sha, *commit.parents)}, QUEUE_PATH)
    reader = subprocess.Popen(["git", "-C", top, "cat-file", "--batch"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)

    @functools.lru_cache(maxsize=8)
    def rows_in(blob):
        return rows_at(read_blob(reader, blob)) if blob is not None else {}

    try:
        for commit in missing:
            summaries[commit.sha] = summarize(rows_in(blob_of[commit.sha]), [rows_in(blob_of[p]) for p in commit.parents])
    finally:
        reader.stdin.close()
        reader.wait()


def blob_ids(top, shas, path):
    """`path`'s blob id at every sha in `shas`, `None` where it does not exist there."""
    ids = sorted(shas)
    out = git(top, "cat-file", "--batch-check", input_bytes="".join(f"{sha}:{path}\n" for sha in ids).encode())
    lines = out.decode().splitlines()
    assert len(lines) == len(ids), f"cat-file --batch-check answered {len(lines)} lines for {len(ids)} names"
    result = {}
    for sha, line in zip(ids, lines):
        fields = line.split()
        result[sha] = None if fields[-1] == "missing" else fields[0]
    return result


def read_blob(reader, blob):
    """One blob's bytes, asked of a `git cat-file --batch` kept open."""
    reader.stdin.write(f"{blob}\n".encode())
    reader.stdin.flush()
    header = reader.stdout.readline().split()
    assert len(header) == 3 and header[1] == b"blob", f"cat-file answered {header!r} for the blob {blob}"
    data = reader.stdout.read(int(header[2]) + 1)
    return data[:-1]


def summarize(rows, parents):
    """What one commit did to the rows, read off its own copy and its parents' copies.

    `added` is every row no parent holds, with its text. `closed` is every row
    it closed, done or dropped, that some parent holds and none holds closed,
    with how it closed and its text there. The rules read nothing else of a
    commit's copy, so a summary stands in for the copy itself.
    """
    added = {row_id: row.text for row_id, row in rows.items() if all(row_id not in parent for parent in parents)}
    closed = {}
    for row_id, row in rows.items():
        if not row.is_closed():
            continue
        before = [parent[row_id] for parent in parents if row_id in parent]
        if before and not any(earlier.is_closed() for earlier in before):
            closed[row_id] = ["done" if row.state.startswith("done:") else "dropped", row.text]
    return {"added": added, "closed": closed}


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
TEXT_LABEL = "the row's text"
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


def description(row_text):
    """A row's description: everything after its five fields."""
    return " ".join(row_text.split()[5:])


def description_words(row_text):
    """The words of a row's description, each word's surrounding punctuation stripped."""
    return [w.strip(".,;:()[]“”") for w in description(row_text).split()]


def is_a_re_file(new_text, candidate_text):
    """§432.1's hand rule: at least half of the candidate's words also appear in the new row's."""
    candidate_words = set(description_words(candidate_text))
    if not candidate_words:
        return False
    new_words = set(description_words(new_text))
    return sum(1 for word in candidate_words if word in new_words) / len(candidate_words) >= 0.5


class Graph:
    """Every row's resolved parent, without and with the sources §433.6 reads out of
    row text, and the tree each first row or §433 source grows either way.

    `summaries` holds each commit's `summarize`, by commit sha, and is filled
    in here for every commit it lacks: a caller that keeps it between runs
    parses only the commits that are new since.
    """

    def __init__(self, top, ref, summaries):
        self.top = top
        self.commits = load_commits(top, ref)
        self.tip_sha = git(top, "rev-parse", ref).decode().strip()
        summarize_missing(top, self.commits, summaries)
        self.summaries = summaries
        self._tip_rows = None
        self.added_at = {}  # row id -> Commit
        self.closed_at = defaultdict(list)  # commit sha -> [row id]
        self.closed_kind = {}  # row id -> "done" | "dropped"
        self.closed_commit = {}  # row id -> Commit
        self.parent_of = {}  # row id -> ParentInfo, from its word or the commit that added it
        self.read_from_text = {}  # row id -> ParentInfo of the input its own text names instead
        self._find_added()
        self._find_closed()
        self._resolve_parents()
        self._read_sources_from_text(KnownInputs(top, ref))
        self.without_text = Tree(self.parent_of)
        self.with_text = Tree({**self.parent_of, **self.read_from_text})

    def tip_rows(self):
        if self._tip_rows is None:
            self._tip_rows = rows_at(load_blobs(self.top, [self.tip_sha], QUEUE_PATH)[self.tip_sha])
        return self._tip_rows

    def is_open_at_tip(self, row_id):
        row = self.tip_rows().get(row_id)
        return row is not None and not row.is_closed()

    def added_text(self, row_id):
        """A row's text in the commit that added it."""
        return self.summaries[self.added_at[row_id].sha]["added"][row_id]

    def _closed_text(self, commit, row_id):
        return self.summaries[commit.sha]["closed"][row_id][1]

    def _find_added(self):
        candidates = defaultdict(list)
        for commit in self.commits:
            for row_id in self.summaries[commit.sha]["added"]:
                candidates[row_id].append(commit)
        for row_id, commits in candidates.items():
            self.added_at[row_id] = min(commits, key=lambda c: c.date)

    def _find_closed(self):
        for commit in self.commits:
            for row_id, (kind, _) in self.summaries[commit.sha]["closed"].items():
                if row_id in self.closed_commit:
                    continue
                self.closed_at[commit.sha].append(row_id)
                self.closed_kind[row_id] = kind
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
            row_text = self.added_text(row_id)
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
            if token_present(cid, row_text) or token_present(row_id, self._closed_text(commit, cid))
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
        candidate_text = self._closed_text(commit, candidate)
        relation = RE_FILE if candidate in dropped and is_a_re_file(row_text, candidate_text) else LED_TO
        return ParentInfo(candidate, label, relation)

    def _read_sources_from_text(self, inputs):
        """§433.6: give a row filed before Q1703 merged the input its own text names.

        Only a row with no word, `from/-` or `from/<id>` is read: `was/<id>`
        re-files a row, and `from/fuzz:` or `from/step:` names the input
        already. The input beats the row that was running (§433.2), but not a
        row in the input's own tree: a row whose parent is the input, or hangs
        below it, was filed by the work on a fault that input showed, and
        §433.2 keeps it there. So rows are read oldest first, each parent's
        own source settled before its children are looked at.
        """
        filed_once_a_word_could_say_it = self._commits_from_q1703_on()
        walk_order = {commit.sha: i for i, commit in enumerate(self.commits)}
        parent_of = dict(self.parent_of)
        for row_id in sorted(self.added_at, key=lambda r: (walk_order[self.added_at[r].sha], row_sort_key(r))):
            commit = self.added_at[row_id]
            if commit.sha in filed_once_a_word_could_say_it:
                continue
            row_text = self.added_text(row_id)
            if not text_may_name_the_source(extract_word(row_text)):
                continue
            source = inputs.source_named_by(description(row_text))
            if source is None or source in parent_and_above(parent_of, parent_of[row_id].parent):
                continue
            parent_of[row_id] = ParentInfo(source, TEXT_LABEL, LED_TO)
            self.read_from_text[row_id] = parent_of[row_id]

    def _commits_from_q1703_on(self):
        """The commit that closed Q1703 done and every commit after it on this ref.

        A row filed in a commit with Q1703's merge in its history could say
        `from/fuzz:` or `from/step:` itself, so its word stands. A row filed
        anywhere else could not, even when its branch merged later. Empty
        where Q1703 is not closed done on this ref, so every row is read.
        """
        commit = self.closed_commit.get(ROW_THAT_LET_A_WORD_NAME_AN_INPUT)
        if commit is None or self.closed_kind[ROW_THAT_LET_A_WORD_NAME_AN_INPUT] != "done":
            return set()
        later = git(self.top, "rev-list", "--ancestry-path", f"{commit.sha}..{self.tip_sha}").decode().split()
        return {commit.sha, *later}


def text_may_name_the_source(word):
    """Whether a row's word leaves its source to its text: no word at all, `from/-` or `from/<id>`."""
    if word is None:
        return True
    if CAME_FROM_AN_INPUT.match(word):
        return False
    match = CAME_FROM.match(word)
    return match is None or match.group(1) == "from"


def parent_and_above(parent_of, parent):
    """`parent` and every parent above it, up to a first row or a fuzz target or STEP file."""
    chain = []
    while parent is not None and parent not in chain:
        chain.append(parent)
        info = parent_of.get(parent)
        parent = info.parent if info is not None else None
    return chain


class KnownInputs:
    """Every fuzz target, fuzz seed and STEP corpus file `ref`'s history ever held:
    what a row's text can name as the input that shows its fault (§433.6).

    The whole history and not the tip, since a target's corpus is cut down
    between nights and a file can leave the STEP corpus after a row names it
    (§433.4).
    """

    def __init__(self, top, ref):
        self.targets = sorted(
            {
                path[len(TARGETS_DIR) : -len(".rs")]
                for path in paths_ever_under(top, ref, TARGETS_DIR)
                if path.endswith(".rs") and "/" not in path[len(TARGETS_DIR) :]
            }
        )
        self.targets_of_seed = defaultdict(set)
        for path in paths_ever_under(top, ref, SEEDS_DIR):
            parts = path[len(SEEDS_DIR) :].split("/")
            if len(parts) == 2:
                self.targets_of_seed[parts[1]].add(parts[0])
        self.seeds = sorted(self.targets_of_seed)
        self.corpus_tails = corpus_tails(corpus_paths_ever(top, ref))
        self.corpus_files_named = defaultdict(list)  # a file's own name -> every corpus path ending in it
        for path in self.corpus_tails:
            self.corpus_files_named[path.rsplit("/", 1)[-1]].append(path)

    def source_named_by(self, text):
        """The one fuzz target or corpus file whose input `text` names, as
        `fuzz:<target>` or `step:<file>`, or `None` where it names none or more than one.

        An input counts only where the text says whose it is: a crash file under
        `artifacts/<target>/`, or alone beside the one target the text names; a
        seed whose hash, or its first eight digits or more, is in one target's
        corpus; a corpus file by a name only it has, or by its directory where
        two files share the name. A crash file beside no target or two, or a
        shared name without its directory, says nothing. Nor does a target
        named with no input of its own: Q58 names the writer target, and is no
        fault of it.
        """
        sources = {f"fuzz:{target}" for target in CRASH_FILE_UNDER_ITS_TARGET.findall(text)}
        if CRASH_FILE_ALONE.search(text):
            named = [target for target in self.targets if name_present(target, text)]
            if len(named) == 1:
                sources.add(f"fuzz:{named[0]}")
        for digits in HASH.findall(text):
            targets = self._targets_of_a_seed_starting(digits)
            if len(targets) == 1:
                sources.add(f"fuzz:{targets.pop()}")
        for name in STEP_FILE_NAME.findall(text):
            path = self._corpus_file_called(name, text)
            if path is not None:
                sources.add(f"step:{self.corpus_tails[path].replace(' ', '_')}")
        return sources.pop() if len(sources) == 1 else None

    def _targets_of_a_seed_starting(self, digits):
        targets = set()
        i = bisect.bisect_left(self.seeds, digits)
        while i < len(self.seeds) and self.seeds[i].startswith(digits):
            targets |= self.targets_of_seed[self.seeds[i]]
            i += 1
        return targets

    def _corpus_file_called(self, name, text):
        paths = self.corpus_files_named.get(name, [])
        if len(paths) > 1:
            tails = {path: self.corpus_tails[path] for path in paths}
            paths = [path for path, tail in tails.items() if tail in text or tail.replace(" ", "_") in text]
        return paths[0] if len(paths) == 1 else None


def paths_ever_under(top, ref, directory):
    """Every file path any commit `ref` reaches added, changed or removed under `directory`."""
    out = git(top, "log", "--root", "--full-history", "--format=", "--name-only", "--no-renames", ref, "--", directory)
    return {line for line in out.decode("utf-8", "surrogateescape").splitlines() if line}


def corpus_paths_ever(top, ref):
    """Every path any version of `corpus/manifest.sha256` on `ref` lists."""
    shas = git(top, "log", "--full-history", "--format=%H", ref, "--", MANIFEST_PATH).decode().split()
    paths = set()
    for blob in load_blobs(top, shas, MANIFEST_PATH).values():
        if blob is None:
            continue
        for line in blob.decode("utf-8", "surrogateescape").splitlines():
            match = MANIFEST_LINE.match(line)
            if match:
                paths.add(match.group(1))
    return paths


def corpus_tails(paths):
    """Each corpus path's tail as §433.1 cuts it: after the `/` closest to its end that still
    names one file. `from/step:` writes the tail with each space as `_`."""
    tails = {}
    for path in paths:
        parts = path.split("/")
        for k in range(1, len(parts) + 1):
            tail = "/".join(parts[-k:])
            if sum(1 for other in paths if other == tail or other.endswith("/" + tail)) == 1:
                tails[path] = tail
                break
    return tails


def name_present(name, text):
    """Whether `name` (a fuzz target) appears in `text` as itself, not as part of a longer name."""
    return re.search(r"(?<![0-9A-Za-z_])" + re.escape(name) + r"(?![0-9A-Za-z_])", text) is not None


class Tree:
    """One way of giving each row its parent, and the tree each first row or
    fuzz target or STEP file grows from it."""

    def __init__(self, parent_of):
        self.parent_of = parent_of
        self.children = defaultdict(list)  # parent id (row or "fuzz:"/"step:" node) -> [row id]
        for row_id, info in parent_of.items():
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


WITHOUT_TEXT = "without the sources read from row text"
WITH_TEXT = "with the sources read from row text (§433.6)"


def print_row_lookup(graph, row_id):
    info = graph.with_text.parent_of.get(row_id)
    if info is None:
        print(f"{row_id}: not added by any commit this walk of {graph.tip_sha[:8]} reached")
        return 1
    line = f"{row_id}: {describe(info)}"
    if row_id in graph.read_from_text:
        line += f" (without it: {describe(graph.parent_of[row_id])})"
    print(line)
    return 0


def describe(info):
    parent = info.parent if info.parent is not None else "none"
    return f"parent {parent}, from {info.label}, {info.relation}"


def print_first_row_table(graph, tree, which):
    print(f"## First rows: how far each one's work has grown, {which}")
    print()
    print("| row | descendants | depth | still open |")
    print("|---|---|---|---|")
    for root in sorted(tree.first_rows(), key=row_sort_key):
        subtree = tree.subtree(root)
        open_count = sum(1 for row_id in subtree if graph.is_open_at_tip(row_id))
        print(f"| {root} | {len(subtree)} | {tree.depth(root)} | {open_count} |")
    print()


def print_weekly_table(graph):
    """Printed once: it counts rows added and closed, and reads no row's parent."""
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


def print_source_table(graph, tree, which):
    print(f"## Each fuzz target or STEP file, by the week its faults were filed (§433), {which}")
    print()
    roots = tree.source_roots()
    if not roots:
        print("(no row on this ref has a fuzz target or a STEP file as its parent)")
        print()
        return
    for root in roots:
        direct = tree.children.get(root, [])
        faults = [row_id for row_id in direct if tree.parent_of[row_id].relation != RE_FILE]
        subtree = tree.subtree(root)
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


def print_sources_read_from_text(graph):
    print("## Rows whose source is read from their own text (§433.6)")
    print()
    if not graph.read_from_text:
        print("(no row filed before Q1703 merged names the input that shows its fault)")
        print()
        return
    print("| row | read from its text | without it |")
    print("|---|---|---|")
    for row_id in sorted(graph.read_from_text, key=row_sort_key):
        without = graph.parent_of[row_id]
        parent = without.parent if without.parent is not None else "none"
        print(f"| {row_id} | {graph.read_from_text[row_id].parent} | {parent}, from {without.label} |")
    print()


def main(argv):
    if len(argv) < 2:
        print("usage: lineage.py <repo-top> <ref> [row-id]", file=sys.stderr)
        return 2
    top, ref = argv[0], argv[1]
    row_ids = [a for a in argv[2:] if a]
    graph = Graph(top, ref, {})
    if row_ids:
        return max(print_row_lookup(graph, row_id) for row_id in row_ids)
    print_first_row_table(graph, graph.without_text, WITHOUT_TEXT)
    print_first_row_table(graph, graph.with_text, WITH_TEXT)
    print_weekly_table(graph)
    print_source_table(graph, graph.without_text, WITHOUT_TEXT)
    print_source_table(graph, graph.with_text, WITH_TEXT)
    print_sources_read_from_text(graph)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
