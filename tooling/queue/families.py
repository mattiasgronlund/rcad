"""The families on a queue: open rows that share a test file, as §450.3 rule 1 counts them.

A row names the test file that pins its shape, so rows on one shape share a
`tests/*.rs` file in their write sets (§450.2). Three or more open rows, not
parked, on one file are a family, and a family is a place to trace before any
of its rows is fixed alone.

A trace is a row whose description, after its pin, its tag and the word that
says where it came from, starts `Trace the family on <file>`, as 450.3 rule 2
files one; `<file>` may be more than one `tests/*.rs` path joined by `and`, as
Q2037's trace was, and it covers every row named on any of them. It may be
open or closed: a trace that is done still covers the rows it named. A file
that has had a trace is a family again only once three open rows that no
trace names share it, and then the family is those rows alone, the ones a new
trace would name. A file that has had no trace keeps all its rows, whatever a
trace on another file named.

A row counts as named on a file when its own `from/` or `was/` link, followed
back through as many such links as it takes, reaches a trace on that file or a
row that trace names directly: the trace's own fix rows (`from/<trace>`, by
450.3 rule 4) and a member the trace re-filed (`was/<member>`), however many
hops either takes. A chain that reaches a trace on a *different* file is no
match; the row stays a member of its own file's family.

A row with two entries on one file, `x.rs#a,x.rs#b`, counts once on it. 450.2's
awk line counted it twice.

`crates/rcad-topo/tests/merged.rs`, `crates/rcad-geom/tests/merged.rs` and
`crates/rcad-step/tests/merged.rs` are never the shared file (§373.7). A lane
that adds a new file under one of those three `tests/` directories also
writes two lines into its `merged.rs`, naming the entry as the whole file or
as an item, `merged.rs#<module>`; that entry says where the file is declared,
not what shape it is on, so it does not belong in any family. Counting it put
six unrelated rows into one family at `ce88b698`: Q2344, Q2346, Q2353, Q2365,
Q2369 and Q2386.
"""

import re

from rows import CLASS, SMALL, PIN

FAMILY = 3
TEST_FILE = re.compile(r"tests/.*\.rs$")
MERGED_RS_FILES = {
    "crates/rcad-topo/tests/merged.rs",
    "crates/rcad-geom/tests/merged.rs",
    "crates/rcad-step/tests/merged.rs",
}
TRACE = ["Trace", "the", "family", "on"]
ROW_ID = re.compile(r"(?<![0-9A-Za-z])[TDQ][0-9]+(?![0-9A-Za-z])")
# Mirrors queue.py's CAME_FROM (queue.py:492): families.py cannot import it,
# since queue.py imports families.py. Keep the two in step.
CAME_FROM = re.compile(r"^(from|was)/([TDQ][0-9]+|-)$")


def families(queue):
    """Each family on `queue` as `(test file, member rows)`, the largest first, then by file.

    The members are in id order, and a family that has had a trace lists only
    the rows whose from/was chain does not reach it.
    """
    traces = [(row.id, *trace) for row in queue.rows for trace in [trace_of(row)] if trace is not None]
    traced_files = {file for _, files, _ in traces for file in files}
    sinks_by_file = {}
    for trace_id, files, named in traces:
        for file in files:
            sinks_by_file.setdefault(file, set()).update(named)
            sinks_by_file[file].add(trace_id)
    found = []
    for file, members in members_by_file(queue).items():
        if file in traced_files:
            sinks = sinks_by_file.get(file, set())
            members = [row for row in members if not reaches_a_sink(row, sinks, queue)]
        if len(members) >= FAMILY:
            found.append((file, sorted(members, key=id_order)))
    return sorted(found, key=lambda family: (-len(family[1]), family[0]))


def members_by_file(queue):
    """Every open row that is not parked, under each test file its write set names, once per file.

    Each file in `MERGED_RS_FILES` is left out: a row names it only to
    declare a new file there, never as the shape it is on.
    """
    by_file = {}
    for row in queue.rows:
        if row.is_closed() or row.state.startswith("parked:") or row.write_set == "-":
            continue
        files = {entry.partition("#")[0] for entry in row.write_set.split(",")}
        for file in files:
            if file not in MERGED_RS_FILES and TEST_FILE.search(file):
                by_file.setdefault(file, []).append(row)
    return by_file


def trace_of(row):
    """The files a trace row is on and the ids it names after them, or None for a row that is no trace.

    A trace may name more than one file, joined by `and`, as Q2037's did:
    `Trace the family on a.rs and b.rs: Q493, Q861, Q1905`. Every
    `tests/*.rs`-looking word up to and including the one carrying the colon
    is a file; a word that is not one, such as `and`, is skipped.
    """
    words = row.fields[5:]
    if words and PIN.match(words[0]):
        words = words[1:]
    if words and (CLASS.match(words[0]) or words[0] == SMALL):
        words = words[1:]
    if words and words[0].startswith(("from/", "was/")):
        words = words[1:]
    if words[:len(TRACE)] != TRACE or len(words) == len(TRACE):
        return None
    rest = words[len(TRACE):]
    files = []
    i = 0
    while i < len(rest):
        word = rest[i]
        stripped = word.rstrip(":,.;")
        if TEST_FILE.search(stripped):
            files.append(stripped.partition("#")[0])
        i += 1
        if word.endswith(":"):
            break
    if not files:
        return None
    return frozenset(files), set(ROW_ID.findall(" ".join(rest[i:])))


def reaches_a_sink(row, sinks, queue):
    """Whether `row`, or a row reached by following its own from/was link back
    through as many more such links as it takes, has an id in `sinks`.

    A cycle stops the walk rather than looping it; nothing here guarantees the
    queue has none.
    """
    seen = set()
    current = row
    while current.id not in seen:
        seen.add(current.id)
        if current.id in sinks:
            return True
        target_id = own_link(current)
        if target_id is None:
            return False
        target = queue.find(target_id)
        if target is None:
            return False
        current = target
    return False


def own_link(row):
    """The id `row`'s own `from/` or `was/` word names, or None for `from/-` or a row with no such word.

    Strips a pin and then a class or `small` tag first, the same order
    `trace_of` strips them, since the link word always comes right after both.
    """
    words = row.fields[5:]
    if words and PIN.match(words[0]):
        words = words[1:]
    if words and (CLASS.match(words[0]) or words[0] == SMALL):
        words = words[1:]
    if not words:
        return None
    link = CAME_FROM.match(words[0])
    if not link or link.group(2) == "-":
        return None
    return link.group(2)


def id_order(row):
    return row.id[0], int(row.id[1:])


def family_line(file, members):
    """One family as 450.2's awk line printed it: how many, the file, and each member with its tier."""
    return " ".join([str(len(members)), file, *(f"{row.id}({row.tier})" for row in members)])
