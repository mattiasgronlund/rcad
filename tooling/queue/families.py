"""The families on a queue: open rows that share a test file, as §450.3 rule 1 counts them.

A row names the test file that pins its shape, so rows on one shape share a
`tests/*.rs` file in their write sets (§450.2). Three or more open rows, not
parked, on one file are a family, and a family is a place to trace before any
of its rows is fixed alone.

A trace is a row whose description, after its pin, its tag and the word that
says where it came from, starts `Trace the family on <file>`, as 450.3 rule 2
files one. It may be open or closed: a trace that is done still covers the rows
it named. A file that has had a trace is a family again only once three open
rows that no trace names share it, and then the family is those rows alone,
the ones a new trace would name. A file that has had no trace keeps all its
rows, whatever a trace on another file named.

A row with two entries on one file, `x.rs#a,x.rs#b`, counts once on it. 450.2's
awk line counted it twice.
"""

import re

from rows import CLASS, SMALL, PIN

FAMILY = 3
TEST_FILE = re.compile(r"tests/.*\.rs$")
TRACE = ["Trace", "the", "family", "on"]
ROW_ID = re.compile(r"(?<![0-9A-Za-z])[TDQ][0-9]+(?![0-9A-Za-z])")


def families(queue):
    """Each family on `queue` as `(test file, member rows)`, the largest first, then by file.

    The members are in id order, and a family that has had a trace lists only
    the rows no trace has named.
    """
    traces = [trace for trace in (trace_of(row) for row in queue.rows) if trace is not None]
    traced_files = {file for file, _ in traces}
    named = {row_id for _, row_ids in traces for row_id in row_ids}
    found = []
    for file, members in members_by_file(queue).items():
        if file in traced_files:
            members = [row for row in members if row.id not in named]
        if len(members) >= FAMILY:
            found.append((file, sorted(members, key=id_order)))
    return sorted(found, key=lambda family: (-len(family[1]), family[0]))


def members_by_file(queue):
    """Every open row that is not parked, under each test file its write set names, once per file."""
    by_file = {}
    for row in queue.rows:
        if row.is_closed() or row.state.startswith("parked:") or row.write_set == "-":
            continue
        files = {entry.partition("#")[0] for entry in row.write_set.split(",")}
        for file in files:
            if TEST_FILE.search(file):
                by_file.setdefault(file, []).append(row)
    return by_file


def trace_of(row):
    """The file a trace row is on and the ids it names after the file, or None for a row that is no trace."""
    words = row.fields[5:]
    if words and PIN.match(words[0]):
        words = words[1:]
    if words and (CLASS.match(words[0]) or words[0] == SMALL):
        words = words[1:]
    if words and words[0].startswith(("from/", "was/")):
        words = words[1:]
    if words[:len(TRACE)] != TRACE or len(words) == len(TRACE):
        return None
    file = words[len(TRACE)].rstrip(":,.;").partition("#")[0]
    return file, set(ROW_ID.findall(" ".join(words[len(TRACE) + 1:])))


def id_order(row):
    return row.id[0], int(row.id[1:])


def family_line(file, members):
    """One family as 450.2's awk line printed it: how many, the file, and each member with its tier."""
    return " ".join([str(len(members)), file, *(f"{row.id}({row.tier})" for row in members)])
